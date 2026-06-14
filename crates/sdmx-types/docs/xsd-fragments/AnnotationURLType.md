<details>
<summary>XSD contract: <code>AnnotationURLType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="AnnotationURLType">
		<xs:annotation>
			<xs:documentation>AnnotationURLType defines an external resource. These can indicate localisation by specifying a language for the resource.</xs:documentation>
		</xs:annotation>
		<xs:simpleContent>
			<xs:extension base="xs:anyURI">
				<xs:attribute ref="xml:lang" use="optional">
					<xs:annotation>
						<xs:documentation>Indicates the language of the resources at the URL, if it is localised. If this does not exist, the resource is not localised.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:simpleContent>
	</xs:complexType>
```

</details>
