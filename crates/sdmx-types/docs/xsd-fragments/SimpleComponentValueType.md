<details>
<summary>XSD contract: <code>SimpleComponentValueType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="SimpleComponentValueType">
		<xs:annotation>
			<xs:documentation>SimpleValueType contains a simple value for a component, and if that value is from a code list, the ability to indicate that child codes in a simple hierarchy are part of the value set of the component for the region.</xs:documentation>
		</xs:annotation>
		<xs:simpleContent>
			<xs:extension base="xs:string">
				<xs:attribute name="cascadeValues" type="common:CascadeSelectionType" use="optional" default="false">
					<xs:annotation>
						<xs:documentation>The cascadeValues attribute, if true, indicates that if the value is taken from a code all child codes in a simple hierarchy are understood be included in the region.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute ref="xml:lang" use="optional">
					<xs:annotation>
						<xs:documentation>The xml:lang attribute specifies a language code for the value. This is used when the component value support multi-lingual values.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
				<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="optional"/>
				<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="optional"/>
			</xs:extension>
		</xs:simpleContent>
	</xs:complexType>
```

</details>
