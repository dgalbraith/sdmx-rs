<details>
<summary>XSD contract: <code>MetadataAttributeUsageType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="MetadataAttributeUsageType">
		<xs:annotation>
			<xs:documentation>MetadataAttributeUsageType defines the structure of how a metadata attribute is used in a data structure. This is a local reference to a metadata attribute from the metadata structure referenced by the data structure. An attribute relationship can be defined in order to describe the relationship of the metadata attribute to the data structure components.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:extension base="MetadataAttributeUsageBaseType">
				<xs:sequence>
					<xs:element name="MetadataAttributeReference" type="common:NCNameIDType">
						<xs:annotation>
							<xs:documentation>MetadataAttributeReference is a local reference to a metadata attribute defined in the metadata structure referenced by this data structure.</xs:documentation>
						</xs:annotation>
					</xs:element>
					<xs:element name="AttributeRelationship" type="AttributeRelationshipType">
						<xs:annotation>
							<xs:documentation>AttributeRelationship defines the relationship between the referenced metadata attribute and the components of the data structure.</xs:documentation>
						</xs:annotation>
					</xs:element>
				</xs:sequence>
			</xs:extension>
		</xs:complexContent>
	</xs:complexType>
```

</details>
